#![no_std]
//! Wire formats shared by RP1 firmware and Linux. All integers are little endian.

pub mod note;
pub mod shm;
pub use rp1_macros::Message;

pub const SRAM_START: u32 = 0x2000_0000;
pub const IMAGE_END: u32 = 0x2000_c000;
pub const LINK_START: u32 = IMAGE_END;
pub const LINK_LEN: usize = 0x3f00;
pub const RESERVED_START: u32 = 0x2000_ff00;
pub const ISRAM_START: u32 = 0x1000_0000;
pub const DSRAM_START: u32 = 0x1000_2000;
pub const LOCAL_LEN: u32 = 8192;
/// Linux view of the link page (BAR2 + 0xc000) as assigned on rpi-cm5 (HW 2026-10-28).
pub const LINK_PHYS: u64 = 0x1f_0040_c000;
/// Loader -> firmware handoff words live at the start of the link page (see [`link`]).
pub const HANDOFF: u32 = LINK_START;

/// Fixed words in the first cache line of the link page (shared SRAM 0x2000_c000).
pub mod link {
    /// Loader writes `HANDOFF_MAGIC`, then the DDR physical address and size of the
    /// shared-memory reservation, over I2C before it starts RP1.
    pub const HANDOFF_MAGIC: usize = 0x00;
    pub const HANDOFF_PHYS_LOW: usize = 0x04;
    pub const HANDOFF_PHYS_HIGH: usize = 0x08;
    pub const HANDOFF_SIZE: usize = 0x0c;
    /// Example doorbell request/reply words (Linux writes request, proc1 echoes reply).
    pub const DOORBELL_REQUEST: usize = 0x10;
    pub const DOORBELL_REPLY: usize = 0x14;
    /// Written by proc0 after `Area::enable`: the published DDR generation (even) or 0.
    pub const SHM_GENERATION: usize = 0x18;
    /// 0 = not enabled yet, 1 = ready, other = error code from enable.
    pub const SHM_STATUS: usize = 0x1c;
    /// Linux requests initialization after PCIe probe; RP1 enables ATU after the final PERSTn.
    pub const SHM_REQUEST: usize = 0x20;
    pub const MAGIC: u32 = u32::from_le_bytes(*b"RP1H");
}

/// rpi-cm5 board table: Linux DDR reservation for `shm`. 2 MiB aligned, above the PoC
/// loader's placements (kernel 0x0600_0000.., initramfs 0x1000_0000.., DTB 0x2000_0000 and
/// its copy ..0x2040_0000) and the base DT's reservations at 0x2100_0000..0x2111_0000.
pub const SHM_PHYS: u64 = 0x2200_0000;
/// Largest reservation the board table allows (the M3 window below is sized for it).
pub const SHM_MAX: u32 = 0x0100_0000;
/// M3 address of the outbound window onto the reservation (PoC-proven base, HW 2026-09-29).
pub const SHM_WINDOW: u32 = 0x8100_0000;

/// A padding-free, fixed-width wire value with no invalid bit patterns.
///
/// # Safety
/// Every bit pattern must be valid, with no padding, references, or pointers.
/// Layout and TYPE_ID must be identical on the host and Cortex-M3.
pub unsafe trait Message: Copy + Send + Sync + 'static {
    const TYPE_ID: u64;
}

/// A type whose shared-SRAM operations never use exclusive load/store or RMW.
///
/// # Safety
/// Implementations must enforce ownership and synchronization without LDREX/STREX.
pub unsafe trait SharedSafe: Sync {}

pub const fn hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0;
    while i < bytes.len() {
        h = (h ^ bytes[i] as u64).wrapping_mul(0x100_0000_01b3);
        i += 1;
    }
    h
}

macro_rules! message_int {
    ($($t:ty),*) => {$ (unsafe impl Message for $t {
        const TYPE_ID: u64 = hash(stringify!($t).as_bytes());
    })*};
}
message_int!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
unsafe impl<T: Message, const N: usize> Message for [T; N] {
    const TYPE_ID: u64 = T::TYPE_ID.wrapping_mul(0x100_0000_01b3) ^ N as u64;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Full,
    Lost(u32),
    Corrupt,
    Invalid,
    Reset,
    NotReady,
    Mismatch,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl core::error::Error for Error {}

/// Stable v2 ownership IDs; do not renumber.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum BlockId {
    Uart0 = 0,
    Uart1,
    Uart2,
    Uart3,
    Uart4,
    Uart5,
    Spi0 = 16,
    Spi1,
    Spi2,
    Spi3,
    Spi4,
    Spi5,
    Spi6,
    Spi7,
    Spi8,
    I2c0 = 32,
    I2c1,
    I2c2,
    I2c3,
    I2c4,
    I2c5,
    I2c6,
    Pwm0 = 48,
    Pwm1,
    I2s0 = 56,
    I2s1,
    I2s2,
    Adc = 64,
    Dma,
    Pio,
}

/// Complete posted writes before publishing metadata or ringing a doorbell.
#[inline]
pub fn barrier() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    #[cfg(target_arch = "arm")]
    unsafe {
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
