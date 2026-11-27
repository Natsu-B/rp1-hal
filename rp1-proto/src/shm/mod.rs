//! Linux shared memory: SPSC rings and blocks over uncached mappings (protocol.md §1–§5).
//!
//! Both peers use these types on raw pointers into mappings that are *not* ordinary memory:
//! on Linux a UIO mapping (arm64 Device attributes: unaligned access and `DC ZVA` fault), on
//! RP1 the outbound PCIe window (every read is a non-posted PCIe round trip). Therefore:
//!
//! - every access is an aligned volatile 32-bit (or, on 64-bit hosts, 64-bit) load/store;
//!   no `memcpy`/`memset` and no Rust reference into a mapping ever exists;
//! - "reader-local" placement: each side only *writes* the peer's memory and only *reads*
//!   its own. A ring is split into `data` (slots, `head`, `dropped`; local to the reader)
//!   and `ctrl` (`tail`, `waiting`; local to the writer). Each endpoint keeps a private
//!   shadow of the index it owns and reads the remote copy only once, in `attach`;
//! - no exclusive (LDREX/STREX) or RMW access; every shared word has a single writer.
//!
//! This module holds the layout constants; the mapping accessors are in `access`, the ring
//! ends in `ring`, the sleep/notify handshake in `wait`, the DMA producer (feature
//! `unstable-ring-dma`) in `staged`.
mod access;
mod block;
mod header;
mod ring;
#[cfg(feature = "unstable-ring-dma")]
mod staged;
#[cfg(test)]
mod tests;
mod wait;
pub mod wake;

pub use access::{copy_from_mapping, copy_to_mapping};
pub use block::{Block, BlockReader, BlockWriter};
pub use header::{Header, HeaderInit, HeaderInitV2, HeaderV2};
pub use ring::{Overflow, RingRx, RingTx};
#[cfg(feature = "unstable-ring-dma")]
pub use staged::{SlotCopy, Staged};
pub use wait::{Progress, Wait, Waiter, wait_any_with};

/// DDR region header (one per reservation, at offset 0).
pub const HEADER_SIZE: usize = 64;
/// `data` and `ctrl` start with this many bytes of control words (two 64-byte lines).
pub const CONTROL_SIZE: usize = 128;
/// `data`: next slot index the writer will fill.
pub const HEAD: usize = 0;
/// `data`: times `head` wrapped past `u32::MAX` (`overflow = overwrite`; written after `head`).
pub const HEAD_HI: usize = 4;
/// `data`: rejected pushes (`overflow = reject`), observational only.
pub const DROPPED: usize = 64;
/// `ctrl`: next slot index the reader will consume.
pub const TAIL: usize = 0;
/// `ctrl`: times `tail` wrapped past `u32::MAX` (`overflow = overwrite`).
pub const TAIL_HI: usize = 4;
/// `ctrl`: the reader's sleep counter (protocol.md §2.3). Odd = asleep and wants a
/// notification, even = awake. Only the reader writes it, adding 1 at every transition, so
/// consecutive sleeps have different values and the writer notifies at most once per sleep.
/// The counter wraps after 2^31 sleeps: a sleep may then reuse the odd value the writer
/// notified last and get no notification; the reader's bounded re-check covers that.
pub const WAITING: usize = 64;
/// Slot word 1 bit: the writer is rewriting this slot (`overflow = overwrite`).
const BUSY: u32 = 1 << 16;

pub const fn slot_size(elem: usize) -> usize {
    8 + ((elem + 7) & !7)
}
/// Bytes of the reader-local ring part; `None` if it does not fit `usize` (same verdict on
/// 32-bit RP1 and 64-bit hosts once compared against a `u32` region length).
pub const fn ring_body_size(elem: usize, depth: usize) -> Option<usize> {
    let Some(slot) = (elem as u64).checked_add(7) else {
        return None;
    };
    let Some(slots) = (depth as u64).checked_mul(8 + (slot & !7)) else {
        return None;
    };
    let Some(total) = slots.checked_add(CONTROL_SIZE as u64) else {
        return None;
    };
    // Computed in u64 so a 32-bit target cannot wrap; regions are u32-sized anyway.
    if total > u32::MAX as u64 {
        None
    } else {
        Some(total as usize)
    }
}
pub const fn block_size(elem: usize) -> usize {
    8 + ((elem + 7) & !7)
}
