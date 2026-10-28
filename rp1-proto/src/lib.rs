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
pub const DDR_PHYS: u64 = 0x2100_0000;
pub const LINK_PHYS: u64 = 0x1f_0040_c000;
pub const HANDOFF: u32 = LINK_START;

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
